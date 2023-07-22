# -*- mode: gnuplot; -*-

set terminal qt font "Sans,8" size 1920,1080

set style fill solid 0.20 border
set style data histograms
set style histogram clustered gap 1

if (!exists("arg_score1")) arg_score1 = "score.data"
if (!exists("arg_score2")) arg_score2 = "score-1.data"

plot arg_score1 using 2:xtic(1) title arg_score1, \
     arg_score2 using 2:xtic(1) title arg_score2
