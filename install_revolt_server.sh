#!/bin/sh
# install_revolt_server.sh ver. 20220220151816 Copyright 2022 alexx, MIT License
# RDFa:deps="[git curl apt-get sudo]"
usage(){ echo "Usage: $(basename $0) [-h]\n\t -h This help message"; exit 0;}
[ "$1" ]&& echo "$1"|grep -q '\-h' && usage

# TODO make this a single cut-n-paste install
# curl --proto '=https' --tlsv1.2 -sSfL https://raw.githubusercontent.com/alexxroche/delta/master/install_revolt_server.sh | sh
# TODO

# fetch the code
git clone https://github.com/revoltchat/delta.git
cd delta

# check rust is installed
which rustc|grep -q cargo || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

if [ -f "$HOME/.bashrc" ]; then
    grep -q cargo ~/.bashrc || cat >>~/.bashrc<<EOF
########
# rust #
########
source $HOME/.cargo/env

EOF
else
    echo "[warn] you should add \`source $HOME/.cargo/env\` to your ~/.\${shell}rc "
fi

# use nightly
if [ ! -f "rust-toolchain.toml" ]; then
    cat >>rust-toolchain.toml<<EOF
[toolchain]
channel = "nightly"
EOF
fi

# install dependencies

### Redis
grep -q Debian /etc/issue || {
    which redis-server|grep -q bin || sudo apt-get install redis-server
}
redis-cli ping | grep -q PONG || { echo "[error] redis is not responding"; exit 3; }
echo "REVOLT_REDIS_URI=redis://localhost:6379" >> .env

### MongoDB on debian
grep -q Debian /etc/issue || {
which gpg|grep -q bin || sudo apt-get install gnupg
wget -qO - https://www.mongodb.org/static/pgp/server-5.0.asc | sudo apt-key add -
# Debian 10 "Buster"
echo "deb http://repo.mongodb.org/apt/debian buster/mongodb-org/5.0 main" | sudo tee /etc/apt/sources.list.d/mongodb-org-5.0.list
sudo apt-get update
sudo apt-get install -y mongodb-org
}

# enable mongo as a service
if ps --no-headers -o comm 1|grep -q 'systemd'; then
    sudo systemctl daemon-reload
    sudo systemctl enable mongod
    sudo systemctl start mongod

  # double check that it is running
  sudo systemctl is-active mongod |grep -q active|| {
      echo "[error] unable to locate mongoDB service"
      exit 2
  }
elif ps --no-headers -o comm 1|grep -q 'init'; then
    sudo service mongod start
    sudo service mongod status
else
    echo "[error] unable to detect if you are using systemd or init"
    exit 1
fi

# fetch an example .env file...
[ -f .env ]|| curl --proto '=https' --tlsv1.2 -sSfL https://github.com/revoltchat/self-hosted/raw/master/.env.example -o .env

# ... orcreate some example ENV variables
[ -f .env ]||{
cat >>.env<<EOF
REVOLT_APP_URL=chat.example.com
REVOLT_MONGO_URI=mongodb://[::1]:27017
REVOLT_PUBLIC_URL=https://chat.isobel.ml/
REVOLT_EXTERNAL_WS_URL=
EOF
}



cat >>/tmp/nginx_revolt.conf<<EOF
map $http_host $revolt_upstream {
  example.com http://127.0.0.1:5000;
  api.example.com http://127.0.0.1:8000;
  ws.example.com http://127.0.0.1:9000;
  autumn.example.com http://127.0.0.1:3000;
  january.example.com http://127.0.0.1:7000;
  vortex.example.com http://127.0.0.1:8080;
}

server {
  listen 80;
  listen 443 ssl http2;
  server_name example.com *.example.com;

  # SSL 

  if ($http_upgrade) {
    # Here, the path is used to reverse the generation of ws. Just roll the keyboard to prevent conflicts with other services.
    rewrite ^(.*)$ /ws_78dd759593f041bc970fd7eef8b0c4af$1;
  }

  location / {
    proxy_pass $revolt_upstream;
    proxy_set_header Host $host;
  }

  location /ws_78dd759593f041bc970fd7eef8b0c4af/ {
    # Note that here is the trailing slash.
    proxy_pass $revolt_upstream/;
    proxy_http_version 1.1;
    proxy_set_header Host $host;
    proxy_set_header Connection $http_connection;
    proxy_set_header Upgrade $http_upgrade;
    # Important, to prevent ws from sending data for a long time and causing timeout disconnection.
    proxy_read_timeout 24h;
  }
}
EOF
sudo mv /tmp/nginx_revolt.conf /etc/nginx/sites-available/chat.example.com


# generate VAPID keys
[ -f vapid_private.pem ]|| openssl ecparam -name prime256v1 -genkey -noout -out vapid_private.pem
[ -f vapid_public.asc ]|| openssl ec -in vapid_private.pem -outform DER|tail -c 65|base64|tr '/+' '_-'|tr -d '\n' >> vapid_public.asc

# and install them in .env
grep -q '^REVOLT_VAPID_PRIVATE_KEY' .env && \
sed -i "s/^REVOLT_VAPID_PRIVATE_KEY.*/REVOLT_VAPID_PRIVATE_KEY=$(base64 -w0 vapid_private.asc)/" .env || \
echo "REVOLT_VAPID_PRIVATE_KEY=$(base64 -w0 vapid_private.asc)/" >> .env

grep -q '^REVOLT_VAPID_PUBLIC_KEY' .env && \
sed -i  "s/^REVOLT_VAPID_PUBLIC_KEY=.*/REVOLT_VAPID_PUBLIC_KEY=$(cat vapid_public.asc)/" .env || \
echo "REVOLT_VAPID_PUBLIC_KEY=$(cat vapid_public.asc)/" >> .env

install_vortex(){
  # revolt uses vortex to provice voice services
  git clone https://github.com/revoltchat/vortex
  cd vortex
  cargo build
  # Set the environment variables as described below
  cargo run
}

# run the actual server
cargo run --release --bin revolt

### Also See
# https://api.revolt.chat/

