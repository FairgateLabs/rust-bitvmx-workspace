docker run --name bitcoin-regtest -d -p 18443:18443 -e BITCOIN_DATA=/data ruimarinho/bitcoin-core \
        -regtest=1 \
        -printtoconsole \
        -rpcallowip=0.0.0.0/0 \
        -rpcbind=0.0.0.0 \
        -rpcuser=foo \
        -rpcpassword=rpcpassword \
        -server=1 \
        -txindex=1 \
        -fallbackfee=0.0002