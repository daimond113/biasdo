use crate::middleware::Identity;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::collections::HashSet;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, TS)]
pub enum ReadWrite {
	Read,
	Write,
}

impl std::ops::Not for ReadWrite {
	type Output = Self;

	fn not(self) -> Self::Output {
		match self {
			ReadWrite::Read => ReadWrite::Write,
			ReadWrite::Write => ReadWrite::Read,
		}
	}
}

macro_rules! scopes {
    {
        $enum_name:ident,
        @impl [] -> {
            scope_enum: [$($scope_enum:tt)*],
            display_impl: [$($display_impl:tt)*],
            from_str_impl: [$($from_str_impl:tt)*],
            access_impl: [$($access_impl:tt)*],
            not_impl: [$($not_impl:tt)*],
        }
    } => {
            #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, SerializeDisplay, DeserializeFromStr, TS)]
            #[ts(export)]
            pub enum $enum_name {
                $($scope_enum)*
            }

            impl $enum_name {
                pub fn access(&self) -> Option<ReadWrite> {
                    match self {
                        $($access_impl)*
                        #[allow(unreachable_patterns)]
                        _ => None
                    }
                }
            }

            impl std::ops::Not for $enum_name {
                type Output = Self;

                fn not(self) -> Self::Output {
                    match self {
                        $($not_impl)*
                        #[allow(unreachable_patterns)]
                        _ => self
                    }
                }
            }

            impl std::fmt::Display for $enum_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let s = match self {
                        $($display_impl)*
                    };
                    write!(f, "{}", s)
                }
            }

            impl std::str::FromStr for $enum_name {
                type Err = String;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    match s {
                        $($from_str_impl)*
                        _ => Err(format!("Unknown scope: {}", s)),
                    }
                }
            }
    };

    {
        $enum_name:ident,
        @impl [
            $scope:ident = $name:literal,
            $($tail:tt)*
        ] -> {
            scope_enum: [$($scope_enum:tt)*],
            display_impl: [$($display_impl:tt)*],
            from_str_impl: [$($from_str_impl:tt)*],
            access_impl: [$($access_impl:tt)*],
            not_impl: [$($not_impl:tt)*],
        }
    } => {
        scopes! {
            $enum_name,
            @impl [$($tail)*] -> {
                scope_enum: [
                    $($scope_enum)*
                    $scope,
                ],
                display_impl: [
                    $($display_impl)*
                    Self::$scope => concat!($name, ".read"),
                ],
                from_str_impl: [
                    $($from_str_impl)*
                    concat!($name, ".read") => Ok(Self::$scope),
                ],
                access_impl: [
                    $($access_impl)*
                ],
                not_impl: [
                    $($not_impl)*
                ],
            }
        }
    };

    {
        $enum_name:ident,
        @impl [
            mut $scope:ident = $name:literal,
            $($tail:tt)*
        ] -> {
            scope_enum: [$($scope_enum:tt)*],
            display_impl: [$($display_impl:tt)*],
            from_str_impl: [$($from_str_impl:tt)*],
            access_impl: [$($access_impl:tt)*],
            not_impl: [$($not_impl:tt)*],
        }
    } => {
        scopes! {
            $enum_name,
            @impl [$($tail)*] -> {
                scope_enum: [
                    $($scope_enum)*
                    $scope(ReadWrite),
                ],
                display_impl: [
                    $($display_impl)*
                    Self::$scope(ReadWrite::Read) => concat!($name, ".read"),
                    Self::$scope(ReadWrite::Write) => concat!($name, ".write"),
                ],
                from_str_impl: [
                    $($from_str_impl)*
                    concat!($name, ".read") => Ok(Self::$scope(ReadWrite::Read)),
                    concat!($name, ".write") => Ok(Self::$scope(ReadWrite::Write)),
                ],
                access_impl: [
                    $($access_impl)*
                    Self::$scope(access) => Some(*access),
                ],
                not_impl: [
                    $($not_impl)*
                    Self::$scope(access) => Self::$scope(!access),
                ],
            }
        }
    };

    { $enum_name:ident, $($body:tt)* } => {
        scopes! {
            $enum_name,
            @impl [$($body)*] -> {
                scope_enum: [],
                display_impl: [],
                from_str_impl: [],
                access_impl: [],
                not_impl: [],
            }
        }
    };
}

scopes! {
	Scope,
	mut Profile = "profile",
	mut Servers = "servers",
	mut Messages = "messages",
	mut Friends = "friends",
}

pub fn has_scope(scopes: &HashSet<Scope>, scope: Scope) -> bool {
	if scopes.contains(&scope) {
		true
	} else if let Some(access) = scope.access() {
		// if the user has Write access, they can also read
		access == ReadWrite::Read && scopes.contains(&!scope)
	} else {
		false
	}
}

impl Identity {
	pub fn is_user_like_with_scope(&self, scope: Scope) -> Option<u64> {
		match self {
			Identity::User(user_id) => Some(*user_id),
			Identity::UserByClient((user_id, scopes)) => {
				has_scope(scopes, scope).then_some(*user_id)
			}
			_ => None,
		}
	}
}
