FROM cosmwasm/cw-gitpod-base:v0.16

RUN cd /workspace && \
    git clone https://github.com/CosmosContracts/juno && \
    cd juno && \
    git checkout v11.0.3 && \
    make install
