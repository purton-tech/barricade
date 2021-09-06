# Git aliases.
alias gst='git status'
alias gcm='git checkout master'
alias c=clear
alias gp='git push'
alias gcam='git commit -a -m'
alias gpsup='git push --set-upstream origin $(git_current_branch)'
alias gcb='git checkout -b'
alias gitsetup='git config --global user.name $NAME && git config --global user.email $EMAIL && mkdir -p ~/.ssh && cp -u /home/host-ssh/id_rsa ~/.ssh && chmod 600 ~/.ssh/id_rsa && ssh-keygen -y -f ~/.ssh/id_rsa > ~/.ssh/id_rsa.pub'

# Cargo watch
alias cwe='USER_TABLE_NAME=keypair_users AUTH_TYPE=encrypted cargo watch --no-gitignore -i *.scss -i *.ts -i "package*" -x fmt -x clippy -x run'
alias cwb='USER_TABLE_NAME=bcrypt_users AUTH_TYPE=normal cargo watch --no-gitignore -i *.scss -i *.ts -i "package*" -x fmt -x clippy -x run'

# Permissions
alias p='sudo chmod 777 /var/run/docker.sock'


# SAST
alias salus='docker run --rm -t -v $HOST_PROJECT_PATH:/home/repo coinbase/salus'

# Database migrations
alias mr='diesel migration run'
alias mre='diesel migration redo'
alias ml='diesel migration list'
alias db='psql $DATABASE_URL'
# Leave a line below or the files will cat together

