#!/usr/bin/fish

for i in (seq 0 100)
    wsk action invoke run_application --param-file $argv[1]
end
