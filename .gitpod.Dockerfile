FROM cosmwasm/cw-gitpod-base:v0.16

RUN export GOPATH=$HOME/go-path && \
    export GOROOT=/home/gitpod/go && \
    export GO_VERSION=1.19.3 && \
    cd /home/gitpod && \
    /usr/bin/git clone https://github.com/CosmosContracts/juno && \
    cd juno && \
    /usr/bin/git checkout v11.0.3 && \
    /usr/bin/make install