#!/bin/bash
#
# This is a little script for forwarding the c-lightning UNIX domain socket so
# that we can access and control c-lightning on a remote machine from where we
# are.
#
# You need a recent version of ssh installed.
#
# usage: ./fwd-clightning.sh <remote server> [<remote path>]
#
# For my use-case, I run this as:
# ./fwd-clightning.sh root@foobar.style /home/treyzania/.lightning/lightning-rpc
#

if [ -z "$1" ]; then
	echo 'usage: ./fwd-clightning.sh <remote server> [<remote path>]'
	exit 1
fi

lpath=$(pwd)/cl-rpc

rpath=.lightning/lightning-rpc
if [ -n "$2" ]; then
	rpath=$2
fi

rm -f $lpath
ssh -nNT -L $lpath:$rpath $1 &
sshpid=$!

echo pid $sshpid
