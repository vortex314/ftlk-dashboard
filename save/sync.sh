set -x
PI_USER=lieven
PI_HOST=pi1.local
cd ~
mkdir rpi-sysroot rpi-sysroot/usr rpi-sysroot/opt
rsync -avz --rsync-path="sudo rsync" --delete $PI_USER@$PI_HOST:/lib rpi-sysroot
rsync -avz --rsync-path="sudo rsync" --delete $PI_USER@$PI_HOST:/usr/include rpi-sysroot/usr
rsync -avz --rsync-path="sudo rsync" --delete $PI_USER@$PI_HOST:/usr/lib rpi-sysroot/usr
# rsync -avz --rsync-path="sudo rsync" --delete $PI_USER@$PI_HOST:/opt/vc rpi-sysroot/opt
cd
symlinks -rcd rpi-sysroot

