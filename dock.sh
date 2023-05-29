set -x
clear
unset all_proxy 
unset http_proxy 
unset https_proxy 
unset no_proxy
docker build -f Dockerfile.armel . -t limero/armel:latest
