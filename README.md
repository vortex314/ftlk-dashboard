# ftlk-dashboard
Customisable config driven  dashboard for MQTT and Redis, based on ftlk-decl
# Goals
- Create good looking dashboards for IoT and Robot devices 
- these Dashboards should be created without any programming effort by just applying settings in a config file as fltk-decl showcases
- hot-reload so the user has a swift feedback on the look and feel he is creating.
- Purpose is to have industrial looking widgets for gauges, switches , meters, compass,..
- Goal is to subscribe to events and send commands on MQTT via the widgets to the different HW modules
- Preferrably a low-budget raspberry pi is used to drive a monitor for these displays
- 
## Install
cargo install cross --git https://github.com/cross-rs/cross

cargo install cross --git https://github.com/cross-rs/cross

## Possible issues

# cross : 'could not get os and arch '
- start Docker or colima on MacOs
# armhf libraries not usable in ARMV6
- these libraries use a ARMV7 startup dat seg faults on Raspberry Pi1

