# minicycle-rs

minicycle-rs is a very simple webhook-based ci/cd runner.

## Configuration

Copy `minicycle.example.toml` to `minicycle.toml`
and edit to fit your needs.
Further details are explained in the comments
in `minicycle.example.toml`.

## TODO

- Actually validate HMAC
- Tokio is weird. But Warp can be tricked into using
    threads instead of concurrency using `SMOL` crate?
    Need to investigate.

## Acknowledgements

- Thanks to Discord user jesse\_polars on the
    Tokio server for help with mutexes.
- Thanks to YouTube user No Boilerplate for god-tier Rust talks

## License

minicycle-rs is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, version 3 of the License only.

minicycle-rs is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with minicycle-rs. If not, see https://www.gnu.org/licenses/.

---

Copyright (c) 2025, maybetree.

