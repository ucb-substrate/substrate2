You will need to install the open source PDK by cloning the [`skywater-pdk` repo](https://github.com/ucb-substrate/skywater-pdk) and pulling the relevant libraries:

```
git clone https://github.com/ucb-substrate/skywater-pdk.git && cd skywater-pdk
git submodule update --init libraries/sky130_fd_pr/latest
```

Also, ensure that the `SKY130_OPEN_PDK_ROOT` environment variable points to the location of the repo you just cloned.
