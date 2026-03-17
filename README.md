Ruuvi terminal client
=====================

![Screenshot of the app](assets/screenshot.png)

A custom terminal Ruuvi client that uses curses to display Ruuvi sensor data. This only supports custom API for now (https://github.com/joonaskokko/ruuvi-api), support for Ruuvi Gateway is coming later.

Tested on macOS and Linux.

Compiling
---------
`$ cargo build`

Running
-------
`$ cargo run`

Configuration
-------------
When starting the app for the first time, it will ask for the API URL to fetch data from. Then it will save it to `.config/ruuvi-terminal-client/config.yml`. You can override this setting with an ENV variable API_URL at start.