#!/usr/bin/fish

dot -Tpdf (curl localhost:8000/graph 2> /dev/null | psub) -o $argv[1]
