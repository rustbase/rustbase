# 🔗 How to contribute
Every help is welcome! See how to contribute below

# Starting
 1. Fork this project on Github
 2. Make a clone of the created fork repository: `git clone https://github.com/youruser/rustbase.git`
 3. Create a branch to commit your feature or fix: `git checkout -b my-branch`
 4. And make your changes!

## Unit tests
If are you creating a new feature, make sure to create a unit tests to it. To make a unit tests, add the following code in the same file of you feature: 
```rust
// Unit tests
#[cfg(test)]
mod your_feature_tests {
    #[test]
    fn testing_feature() {
        ...
    }
}
```

### Running unit tests
To run a unit test use: `cargo run -- --test-threads=1`

See more about unit tests [here](https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html)

## Commit messages
We suggest that commit messages follow the *conventional commit*.

See about *conventional commit* [here](https://www.conventionalcommits.org/en/v1.0.0/)

# When you're done, make your Pull Request!
  * Commit your changes
  * Push your branch to your fork: `git push origin my-branch`
  * Go to Pull Requests from the root repository and create a [Pull Request](https://github.com/rustbase/rustbase/pulls) with your commit(s)