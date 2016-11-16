# Contributing

Every contribution is highly welcome.

Fork, then clone the repo:

```
git clone git@github.com:your-username/fireplace.git
```

Make your changes, try out your changes and make sure the following applies:

- new features are easily configurable via a struct that implements `Deserialize` via [`serde`](https://serde.rs/)
- is split into a seperate `feature` if it increases compile time substancially and/or will most likely not be used by a large amount of users
- passes `cargo clippy` except for well explained exceptions. To be able to check clippy & building for all configurations use `vagga clippy`. You will need [vagga](https://github.com/tailhook/vagga) on your machine.
- was formated with `rustfmt` and the config in the repository's root
- roughly follows the design of existing code

Push to your fork and submit a pull request.

At this point you will need to wait for a review of your purposed changes and
maybe need to make some additional changes before your code gets merged.

If you are not sure where to start or how your idea fits the project, feel free
to open an issue to discuss the matter.
