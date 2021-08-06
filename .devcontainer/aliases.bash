# Git aliases.
alias gst='git status'
alias gcm='git checkout master'
alias c=clear
alias gp='git push'
alias gcam='git commit -a -m'
alias gpsup='git push --set-upstream origin $(git_current_branch)'
alias gcb='git checkout -b'

# Cargo watch
alias cw='cargo watch --no-gitignore -i *.scss -i *.ts -i "package*" -x fmt -x clippy -x run'

# Permissions
alias p='sudo chown -R vscode . && sudo chmod 777 /var/run/docker.sock'

# Database migrations
alias mr='diesel migration run'
alias mre='diesel migration redo'
alias ml='diesel migration list'
alias db='psql $DATABASE_URL'
# Leave a line below or the files will cat together

