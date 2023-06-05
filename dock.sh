set -x
clear
docker build -f Dockerfile.pi1 . -t pi1
docker run -it --rm --privileged -v $(pwd):$(pwd) --name pi1 pi1 

