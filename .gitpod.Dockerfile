# FROM cosmwasm/cw-gitpod-base:v0.16

### wasmd ###
FROM cosmwasm/wasmd:v0.30.0 as wasmd

### rust-optimizer ###
FROM cosmwasm/rust-optimizer:0.11.5 as rust-optimizer

FROM gitpod/workspace-full:latest

COPY --from=wasmd /usr/bin/wasmd /usr/local/bin/wasmd
COPY --from=wasmd /opt/* /opt/

RUN sudo apt-get update \
    && sudo apt-get install -y jq \
    && sudo rm -rf /var/lib/apt/lists/*

RUN rustup update stable \
   && rustup target add wasm32-unknown-unknown

RUN cd /home/gitpod && \
    wget https://golang.org/dl/go1.19.2.linux-amd64.tar.gz && \
    mkdir /home/gitpod/go1.19.2 && \
    tar -C /home/gitpod/go1.19.2 -xzf go1.19.2.linux-amd64.tar.gz && \
    /usr/bin/git clone https://github.com/CosmosContracts/juno && \
    cd juno && \
    /usr/bin/git checkout v11.0.3 && \
    export GOROOT=/home/gitpod/go1.19.2/go && \
    export PATH="${GOROOT}/bin:$PATH" && \
    export GOPATH="${GOROOT}" && \
    export GOVERSION="go1.19.2" && \
    echo "GO=" && which go && \
    /usr/bin/make install && \
    sudo cp /home/gitpod/go1.19.2/go/bin/junod /usr/bin
