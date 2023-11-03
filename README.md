# biasdo

biasdo is an open source chat app made for users, by users.

## Stack

biasdo is built with the following technologies:

- actix-web
- sqlx
- MariaDB
- Svelte
- SvelteKit
- TypeScript
- TailwindCSS

## Host

biasdo is hosted by [fly.io](https://fly.io) and [Vercel](https://vercel.com).

## Self-hosting

A public instance of biasdo is available at [biasdo.daimond113.com](https://biasdo.daimond113.com), and it's api at [biasdo-api.daimond113.com](https://biasdo-api.daimond113.com). If you want to self-host biasdo, you can follow the instructions below.

### Client

1. Clone the repository
2. Go into the `packages/client` directory and create a `.env` file with the following contents:

```sh
VITE_API_URL= # the url of the api, for example https://biasdo-api.daimond113.com (IMPORTANT: do not include a trailing slash)
VITE_APP_NAME=biasdo # although you can change this to whatever you want, it would be nice if you kept it as biasdo
```

3. Install the dependencies
4. Now you can follow the usual steps to run a SvelteKit app, which can be found [here](https://kit.svelte.dev/docs/building-your-app)

### API

1. Clone the repository
2. Go into the `packages/server` directory and create a `.env` file with the following contents:

```sh
DATABASE_URL= # the url of the database, for example mysql://root:password@localhost:3306/biasdo (IMPORTANT: the app uses MariaDB, and will NOT work with MySQL as it uses the RETURNING keyword)
```

3. Install the dependencies
4. Install the sqlx CLI by running `cargo install sqlx-cli`
5. Run `cargo sqlx database create` to create the database
6. Run `cargo sqlx migrate run` to run the migrations
7. Now you can build and run the app like any other cargo project

## Contributing

As the motto says, biasdo is made for users, by users. If you want to contribute, you can do so by opening a pull request. If you want to contribute but don't know how, or report an issue, you can check out the [GitHub issues](https://github.com/daimond113/biasdo/issues).

## License

This project is licensed under the [MIT License](https://choosealicense.com/licenses/mit).
