<br />

<div align="center">

<img src="packages/client/static/logotype.svg" width="200" align="center" alt="biasdo">

</div>

<br />

biasdo is an open source chat app made for users, by users.

## Technologies

biasdo uses technologies made by the open source community, and a
detailed list can be found in the [root package.json](package.json), [client package.json](packages/client/package.json), and [backend Cargo.toml](Cargo.toml), but the core technologies are:

- [SvelteKit](https://kit.svelte.dev) for the frontend
- [actix-web](https://actix.rs) for the backend
- [SQLx](https://github.com/launchbadge/sqlx) for the type-safe database queries
- [MariaDB](https://mariadb.org) as the database

## Public instance

The official public biasdo instance is hosted on a Raspberry PI for the backend and [Vercel](https://vercel.com) for the frontend. The instance is available at [biasdo.daimond113.com](https://biasdo.daimond113.com), and it's api at [biasdo-api.daimond113.com](https://biasdo-api.daimond113.com).

## Self-hosting

### Client

1. Clone the repository
2. Go into the `packages/client` directory and create a `.env` file with the following contents:

```sh
VITE_API_URL= # the url of the api, for example https://biasdo-api.daimond113.com/v0 (IMPORTANT: do not include a trailing slash)
```

3. Install the dependencies
4. Now you can follow the usual steps to run a SvelteKit app, which can be found [here](https://kit.svelte.dev/docs/building-your-app)

### API

1. Clone the repository
2. Create a `.env` file with the following contents:

```sh
DATABASE_URL= # the url of the database, for example mysql://root:password@localhost:3306/biasdo (IMPORTANT: biasdo uses MariaDB, and has not been tested with MySQL)
```

3. Install the dependencies
4. Create the database using `CREATE DATABASE <name in the url>`;
5. Now you can build and run the app like any other Cargo project

## Contributing

As the motto says, biasdo is made for users, by users. If you want to contribute, you can do so by opening a pull request. If you want to contribute but don't know how, or report an issue, you can check out the [GitHub issues](https://github.com/daimond113/biasdo/issues).

## License

This project is licensed under the [MIT License](https://choosealicense.com/licenses/mit).
