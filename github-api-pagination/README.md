# GitHub API pagination
This crate implements an async iterator that reads repositories from a GitHub API using pagination.
The iterator gets data from the API in batches, and then has to return the individual items from each batch one by one.
