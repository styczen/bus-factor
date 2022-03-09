# Bus factor estimation
Project is CLI application to estimate bus factor for on GitHub repositories.

## What we want to get from this project?
Find all GitHub repositories which have [bus factor](https://en.wikipedia.org/wiki/Bus_factor)
equal to **1**. User of this CLI app gives programming language and number
of repositories to fetch. Those repositories should be sorted by amount of stars.
Then all of those repositories' information should be further analyze...

## What it means **bus factor** equal to **1**?
Most active developer's contributions account for **75% or more**
of total contributions count from the top 25 most active developers.

## Steps
1. Fetch repositories with provided programming language and project count
   as command line arguments.
   Sample request: https://api.github.com/search/repositories?q=language:rust&sort=stars&order=desc
   To avoid receiving tons of data, GitHub decided to paginate the response.
   In response header there are links to next and last page. This allows to 
   easily read next page of results and compare amount of already read 
   repositories with project count from CLI arguments.
2. For each repository get all contributors (Sample request: https://api.github.com/repos/denoland/deno/contributors) and check the first 25 most active contributors.
3. For those 25 contributors, check whether the most active one has **75% or more** 
   of all contributions from those 25 most active.
4. Save name of the repository, name of the most active contributor and their
   contribution percentage to print to console.

## How to run

On Ubuntu system you might need to install some dependencies
```
sudo apt install libssl-dev pkg-config
```

If all dependencies are installed, run program
```
GITHUB_ACCESS_TOKEN=<YOUR_GITHUB_TOKEN> cargo run -- --language rust --project_count 50
```

or first export token variable 

```
export GITHUB_ACCESS_TOKEN=<YOUR_GITHUB_TOKEN>
```

and then in the same terminal run the program

```
cargo run -- --language rust --project_count 50
```

## What to do to make this project better
- add more tests,
- better error handling,
- get rid of panic! (but this connects to the previous point).
- add logging.
 