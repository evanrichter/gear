# Build Stage
FROM ghcr.io/evanrichter/cargo-fuzz:latest as builder

## Add source code to the build stage.
ADD . /src
WORKDIR /src

RUN echo building instrumented harnesses && \
    bash -c "pushd utils/economic-checks/fuzz && cargo +nightly -Z sparse-registry fuzz build && popd" && \
    mv utils/economic-checks/fuzz/target/x86_64-unknown-linux-gnu/release/composer_fuzz_target /composer_fuzz_target && \
    mv utils/economic-checks/fuzz/target/x86_64-unknown-linux-gnu/release/simple_fuzz_target /simple_fuzz_target && \
    echo done

RUN echo building non-instrumented harnesses && \
    export RUSTFLAGS="--cfg fuzzing -Clink-dead-code -Cdebug-assertions -C codegen-units=1" && \
    bash -c "pushd utils/economic-checks/fuzz && cargo +nightly -Z sparse-registry build --release && popd" && \
    mv utils/economic-checks/fuzz/target/release/composer_fuzz_target /composer_fuzz_target_no_inst && \
    mv utils/economic-checks/fuzz/target/release/simple_fuzz_target /simple_fuzz_target_no_inst && \
    echo done

# Package Stage
FROM rustlang/rust:nightly

COPY --from=builder /composer_fuzz_target /composer_fuzz_target_no_inst /
COPY --from=builder /simple_fuzz_target /simple_fuzz_target_no_inst /
