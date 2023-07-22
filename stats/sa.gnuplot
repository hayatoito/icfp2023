# -*- mode: gnuplot; -*-

if (!exists("arg_data")) arg_data = "sample.data"

# set xtics 1000000
# set xtics format '' scale 0
set xlabel 'Iteration'

set y2label 'Accept rate'
set y2tics
# set y2range [0:1]

plot arg_data using 1:2 with line title "score", \
     '' using 1:3 with line title "best", \
     '' using 1:6 axis x1y2 with line title "accept rate positive", \
     '' using 1:7 axis x1y2 with line title "accept rate negative"
