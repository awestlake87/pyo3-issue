FROM tensorflow/tensorflow:latest

RUN apt-get update
RUN apt-get install -y curl
RUN apt-get install -y libssl1.0-dev

RUN mkdir -p /opt/.cargo /opt/.multirust
ENV CARGO_HOME=/opt/.cargo
ENV RUSTUP_HOME=/opt/.multirust

RUN chmod 777 $CARGO_HOME

RUN curl https://sh.rustup.rs -sSf \
    | sh -s -- --default-toolchain nightly-2020-10-15 -y
ENV PATH="${PATH}:/${CARGO_HOME}/bin"

RUN rm -rf ${CARGO_HOME}/registry

RUN mkdir /pyo3-issue
WORKDIR /pyo3-issue
