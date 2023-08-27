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

# on Pi
```shell
github clone https://github.com/MoAlyousef/cfltk.git
cd cfltk
mkdir build
cd build
# configure
cmake -Bbin -DOPTION_USE_SYSTEM_LIBPNG=ON -DOPTION_USE_SYSTEM_LIBJPEG=OFF -DOPTION_USE_SYSTEM_ZLIB=OFF -DCFLTK_LINK_IMAGES=ON -DOpenGL_GL_PREFERENCE=GLVND -DOPTION_USE_GL=ON -DCFLTK_USE_OPENGL=ON -DOPTION_USE_PANGO=ON -DCFLTK_SINGLE_THREADED=OFF -DCMAKE_EXPORT_COMPILE_COMMANDS=ON -DCFLTK_CARGO_BUILD=ON -DFLTK_BUILD_EXAMPLES=OFF -DFLTK_BUILD_TEST=OFF -DOPTION_LARGE_FILE=ON -DOPTION_USE_THREADS=ON -DOPTION_BUILD_HTML_DOCUMENTATION=OFF -DOPTION_BUILD_PDF_DOCUMENTATION=OFF -DCMAKE_INSTALL_PREFIX=bin -DCMAKE_BUILD_TYPE=Release
# build
cmake --build bin  --target install # no --parallel : kills the device
```
# on PC
```shell
# copy rpi files
set -x
PI_USER=lieven
PI_HOST=pi1.local
cd ~
mkdir rpi-sysroot rpi-sysroot/usr rpi-sysroot/opt rpi-sysroot/cfltk
rsync -avz --rsync-path="sudo rsync" --delete $PI_USER@$PI_HOST:/lib rpi-sysroot
rsync -avz --rsync-path="sudo rsync" --delete $PI_USER@$PI_HOST:/usr/include rpi-sysroot/usr
rsync -avz --rsync-path="sudo rsync" --delete $PI_USER@$PI_HOST:/usr/lib rpi-sysroot/usr
rsync -avz --rsync-path="sudo rsync" --delete $PI_USER@$PI_HOST:/home/lieven/workspace/cfltk/build/fltk/lib rpi-sysroot/cfltk
rsync -avz --rsync-path="sudo rsync" --delete $PI_USER@$PI_HOST:/home/lieven/workspace/cfltk/build/libcfltk.a rpi-sysroot/cfltk/lib

# rsync -avz --rsync-path="sudo rsync" --delete $PI_USER@$PI_HOST:/opt/vc rpi-sysroot/opt
cd
symlinks -rcd rpi-sysroot

https://github.com/japaric/rust-cross
https://github.com/sdt/docker-raspberry-pi-cross-compiler

# What I will remember from Rust language 
- No native GUI , just unsafe wrappers around existing C/C++ libraries
- Create callbacks is hell , just search google for all the questions on this subject. Basically you need an Arc on a Mtutex on an FnMut that supports Sync and Send and is also static. Rust says : human you're unable to reason about the safety of these callbacks, so I'll forbid to do anything except use message passing channels.
- A callback and nested types is a lot of code noise 
+ Nice build and library system to re-use what others created 
- didn't find an equivalent of libuv that integrates with async/await pattern
- Rust avoid the shoot in the foot scenario's like C++, by parelizing the developer and remove any arms. 

# My struggles to get a GUI 
1. I can use in RUST 
2. that permits to create a highly dynamic composable dashboard
- is config driven
- is hot reloadable
- is composable
- is customisable
- is extensible
- is easy to use
3. uses a pubsub mechanism to communicate with the outside world
4. that has base widgets for :
- gauges
- meters
- switches
- compass
- graphs
- text
- images
- video
- buttons
- sliders
- knobs
- ...
5. runs on Raspberry Pi B if possible 
### Some examples
https://priidash.sourceforge.net/#SetupText
https://images.squarespace-cdn.com/content/v1/5e7135caec915516f2aaf65b/1585586605207-7FLIYPZ93HYFMIFDW0LD/dashboard2.png?format=1500w
https://www.vtscada.com/wp-content/uploads/2015/02/VTScada11_Generator2.png
https://www.vtscada.com/wp-content/uploads/2015/02/VTScada_GeneratorDashboard.png

## What I tried and getting run in circles to find a fit
### FLTK ++
- FLTK is a C++ library that has a Rust wrapper
- very good support by Mo and recent updates
- very good documentation
- very good examples
- limited widgets for what I need
### LVGL ++
- LVGL is a C library that has a Rust wrapper
- very good support and recent updates
- very good documentation
- very good examples
- quirks in RUST as it expects static allocated styles
- extended list of widgets
- meter widget in rust no example as broken
- recompilation fails , breaks easily and is also based on the older lvgl version 0.6 , while version 0.8 is ongoing 
### EGUI
- EGUI is a Rust library
- very good support and recent updates
- limited documentation
- good widgets but not enough
- easy to make it hot reloadable and dynamic, positioning and coloring is a mess
- missing fine-grained control and positioning of widgets
### IUP ???
- IUP is a C library that has a Rust wrapper
### GTK
- GTK is a C library that has a Rust wrapper
- very good support and recent updates
- found some good industrial widgets
- very good documentation
### QT
- QT is a C++ library that has a Rust wrapper
- very good support and recent updates
- very good documentation
- too big to handle
### NANNOU - suggested by Copilot
- NANNOU is a Rust library
- very good support and recent updates
- not impressed by widgets
### IMGUI 
- IMGUI is a C library that has a Rust wrapper
- very good support and recent updates
### DEAR IMGUI
- DEAR IMGUI is a C library that has a Rust wrapper
- very good support and recent updates
### No more suggestion from Copilot

