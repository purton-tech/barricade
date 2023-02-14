# Git aliases.
alias gst='git status'
alias gcm='git checkout master'
alias c=clear
alias gp='git push'
alias gcam='git commit -a -m'
#alias gpsup='git push --set-upstream origin $(git_current_branch)'
alias gpsup="git push --set-upstream origin $(git symbolic-ref -q HEAD | sed -e 's|^refs/heads/||')"
alias gcb='git checkout -b'
alias gitsetup='git config --global user.name $NAME && git config --global user.email $EMAIL && mkdir -p ~/.ssh && cp -u /home/host-ssh/id_rsa ~/.ssh && chmod 600 ~/.ssh/id_rsa && ssh-keygen -y -f ~/.ssh/id_rsa > ~/.ssh/id_rsa.pub'
alias gcr='f() { git checkout -b $1 origin/$1; }; f'

# Cargo watch
alias cwe='USER_TABLE_NAME=keypair_users AUTH_TYPE=encrypted mold -run cargo watch --no-gitignore -i *.scss -i *.ts -i "package*" -x run'
alias cwb='USER_TABLE_NAME=bcrypt_users AUTH_TYPE=normal mold -run cargo watch --no-gitignore -i *.scss -i *.ts -i "package*" -x run'


# Watch
alias watch-app='mold -run cargo watch --workdir /workspace/ -w crates/ui-components -w crates/axum-server -w crates/db -w crates/asset-pipeline/dist --no-gitignore -x "run --bin barricade"'
alias wa=watch-app
alias watch-pipeline='npm run start --prefix /workspace/crates/asset-pipeline'
alias wp=watch-pipeline
alias watch-zola='cd /workspace/www && zola serve --drafts --interface 0.0.0.0 --port 7104 --base-url localhost'
alias wz=watch-zola

# SAST
alias salus='docker run --rm -t -v $HOST_PROJECT_PATH:/home/repo coinbase/salus'

# Database migrations
alias dbmate='dbmate --no-dump-schema --migrations-dir /workspace/crates/db/migrations'
alias db='psql $DATABASE_URL'
# Start node dev i..e parcel
alias nrs='npm run start'

# Leave a line below or the files will cat together

