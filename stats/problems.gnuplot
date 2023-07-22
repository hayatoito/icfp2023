# -*- mode: gnuplot; -*-

set terminal qt font "Sans,8" size 1920,1080

set style fill solid 0.20 border
set style data histograms
set style histogram clustered gap 1

plot 'problems.data' using 2:xtic(1) title "musicions", \
     '' using 3 title "attendees"
