#!/bin/bash
for port in `seq 1 65535`
do
    echo $port
    telnet 192.168.4.1 $port
done