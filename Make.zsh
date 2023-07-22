repo=icfp2023
site='https://api.icfpcontest.com'
bin=./target/release/$repo
curl_options=(--silent --show-error)
max_problem_id=90

api_token() {
  my-passkey $repo
}

prepare() {
  # cd .. && cargo new $repo
  rustup update

  gh repo create --private $repo
  git remote add origin git@github.com:hayatoito/$repo.git

  mkdir -p task/{problem,solution,stats,draw}
}

get_number_of_problems() {
  # In-Contest:
  # curl $curl_options $site/problems | jq .number_of_problems

  # Post-Contest
  echo 90
}

download_problems() {
  local num=$(get_number_of_problems)
  echo "problems: $num"

  cd ./problem
  local next=1
  for i in {$next..$num}; do
    curl -O "https://cdn.icfpcontest.com/problems/$i.json"
    # curl "$site/problem?problem_id=$i" -o raw-$i.json
    # cat raw-$i.json | jq -r .Success > $i.json
  done
}

build() {
  cargo build --release
}

plot_problems() {
  local problems_data=./stats/problems.data
  echo "id musicians attendees" > $problems_data
  for i in {1..$max_problem_id}; do
    echo -n "$i " >> $problems_data
    cat ./problem/$i.json | \
      jq -r '"\(.musicians | length) \(.attendees | length)"' >> $problems_data
  done
  cd ./stats && gnuplot -p ./problems.gnuplot
}

best_score_refresh() {
  build
  $bin best-score-refresh
}

best_score_total() {
  cat ./stats/best-score.json | jq '[.[]] | add'
}

# score_cal_compare() {
#   for i in {1..90}; do
#     local sol=./solution/best/$i.json
#     echo "$i: $($bin score $i $sol) $($bin score2 $i $sol) "
#   done
# }

score() {
  build
  $bin score $1 $2
}

stats_score_update() {
  # Create ./stats/score-{name}.data file

  build
  local d=${1:-./solution/best}
  local score_data=./stats/score-${d:t}.data
  echo "id score" > $score_data
  for i in {1..$(get_number_of_problems)}; do
    local score=$($bin score $i $d/$i.json)
    echo "$i $score" >> $score_data
  done
  ls -l $score_data
}

plot_stats_score() {
  gnuplot -p -e "arg_score1='$1'; arg_score2='$2'" ./stats/score.gnuplot
}

plot_sa() {
  gnuplot -p -e "arg_data='$1'" ./stats/sa.gnuplot
}

draw_problems() {
  build
  for i in {1..$max_problem_id}; do
    $bin plot-problem $i ./draw/problem/$i.svg
  done
}

draw_solutions() {
  build
  local d=${1:-./solution/best}
  mkdir -p ./draw/${d:t}
  for i in {1..$max_problem_id}; do
    $bin draw-solution $i $d/$i.json ./draw/${d:t}/$i.svg
  done
}

solve() {
  build
  time RUST_LOG=info $bin solve $@
}

solve_debug() {
  build
  time RUST_LOG=debug $bin solve ${1:-1}
}

solve_all() {
  build
  for i in {1..$max_problem_id}; do
    time RUST_LOG=info $bin solve ${i}
  done
}

solve_all_parallel() {
  build
  RUST_LOG=info parallel --joblog ./log/joblog --results ./log/results $bin solve {} --initial-solution-path ./solution/best/{}.json ::: {1..$max_problem_id}
}

solve_all_parallel_retry() {
  build
  RUST_LOG=info LANG=C parallel --retry-failed --joblog ./log/joblog
}

solve_basic() {
  build
  time RUST_LOG=info $bin solve-basic ${1:-1}
}

submit() {
  local id=$1
  local file=$2
  local token=$(api_token)
  local jq_args=(
    -n
    --argjson problem_id $id
    --arg contents $(cat $file)
    '{ "problem_id": $problem_id, "contents": $contents}'
  )
  jq ${jq_args} | curl $curl_options --header "Authorization: Bearer ${token}" --json @- $site/submission
}

submit_best() {
  local token=$(api_token)
  for i in {1..$max_problem_id}; do
    if [[ -f ./solution/best/${i}.json ]]; then
      if [[ -f ./solution/submission/${i}.json ]] && cmp --silent ./solution/best/${i}.json ./solution/submission/${i}.json ; then
        echo "Skipping ./solution/best/${i}.json"
      else
        echo "Submitting ./solution/best/${i}.json..."
        submit $i ./solution/best/${i}.json
        cp -a ./solution/best/${i}.json ./solution/submission/
        sleep 1
      fi
    fi
  done

  userboard
}

submissions() {
  local token=$(api_token)
  curl --header "Authorization: Bearer ${token}" "$site/submissions?offset=0&limit=10" \
    | jq .
}

userboard() {
  # api.icfpcontest.com/userboard
  local token=$(api_token)
  curl $curl_options --header "Authorization: Bearer ${token}" $site/userboard \
    | tee ./stats/userboard.json \
    | jq .
}

watch() {
  watchman-make -p 'draw/wip.svg' --run "my-browser reload wip.svg"
}

watch_100() {
  watchman-make -p 'draw/wip.svg' --run "my-browser --port 10028 reload wip.svg"
}

browser() {
  my-google-chrome --port 10028
}

movie() {
  cd ./draw/wip
  ffmpeg -y -framerate 2 -pattern_type glob -i '*.svg' -codec:v vp9 -lossless 1 out.webm
}

bench() {
  build
  hyperfine "$bin bench ${1:-60}"
}

bench_scoring() {
  build
  $bin bench-scoring ${1:-1}
}

profiling() {
  build
  LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libprofiler.so CPUPROFILE=gperf-cpu.prof $bin bench ${1:-60}
}

profiling_web() {
  pprof -http=:8085 $bin ./gperf-cpu.prof
}
