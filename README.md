# Pyo3 Issue

A stripped down example of an issue I've been seeing with pyo3, asyncio, and rust async-std. It's unfortunately a tricky problem.

## Setup

### Install Docker CE

- Install Docker
  ```bash
  sudo apt-get install docker.io
  ```
- Add your user to the docker group
  ```bash
  sudo usermod -aG docker $USER
  ```
- Reboot
  ```bash
  sudo reboot
  ```
- Test Docker CE
  ```bash
  docker run hello-world
  ```

### Make

```
sudo apt-get install make
```

## Running the Test

Run `scripts/test.sh` from the project directory. It should build the docker environment and run the test for you.

```
scripts/test.sh
```

> If you see what I see, then the test itself should pass, but the process should exit with an error similar to this:

```
pyo3-issue$ scripts/test.sh
make: '.make/pyo3-issue-dev' is up to date.
    Finished test [unoptimized + debuginfo] target(s) in 0.08s
     Running target/debug/deps/pyo3_issue-960535bb3b8043d0

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running target/debug/deps/test_pyo3-31dc2767723e4ed7

running 1 test
test test_pyo3_error ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

Exception ignored in: <module 'threading' from '/usr/lib/python3.6/threading.py'>
Traceback (most recent call last):
  File "/usr/lib/python3.6/threading.py", line 1289, in _shutdown
    assert tlock.locked()
AssertionError:
FATAL: exception not rethrown
error: test failed, to rerun pass '--test test_pyo3'

Caused by:
  process didn't exit successfully: `/pyo3-issue/target/debug/deps/test_pyo3-31dc2767723e4ed7` (signal: 6, SIGABRT: process abort signal)
```