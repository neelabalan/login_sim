# Login Attempt Simulator

> This is based on [login attempt simulator](https://github.com/stefmolin/login-attempt-simulator) by stefmolin which is written in Python

> I wanted to re-write the same in Rust

Simulation of regular login activity on a site and random activity from hackers using brute-force password guessing attacks. The login process involves a username and password and no additional validation.


## Usage

The simulator reads configuration from a JSON file passed as a command-line argument.

### Configuration File

Create a `config.json` file with the following structure:

```json
{
  "start_date": "2022-01-01 00:00:00",
  "end_date": "2022-03-13 00:00:00",
  "seed": 13,
  "first_names": ["Kent", "Armando", ...],
  "last_names": ["Daniels", "Holland", ...],
  "output": {
    "userbase": "data/userbase.json",
    "logs": "logs/log.csv",
    "attacks": "logs/attack.csv"
  }
}
```

- `start_date` and `end_date`: Simulation period in `%Y-%m-%d %H:%M:%S` format.
- `seed`: Random seed for reproducibility.
- `first_names` and `last_names`: Arrays of strings for generating user names.
- `output`: Paths for output files.

### Running the Simulator

```shell
cargo run -- config.json
```

This will generate the simulation data and save it to the specified output paths.

## Analysis

The `notebook/anomaly_detection.ipynb` Jupyter notebook contains exploratory data analysis and anomaly detection algorithms to identify suspicious login activity from the generated logs.

### Python Dependencies

Dependencies are managed in `pyproject.toml`. Install with:

```shell
uv install
# or
uv sync
```

## Assumptions

The simulator makes the following assumptions about valid users of the website:

- Valid users come according to a Poisson process with an hourly rate that depends on the day of the week and the time of day. A Poisson process models arrivals per unit time (hour here) as a Poisson distribution with mean λ (lambda) and the interarrival times are exponential distributed with mean 1/λ.
- Valid users connect from 1-3 IP addresses (unique identifier for devices using the Internet), which are 4 random integers in `[0, 255]` separated by periods. It is possible, although highly unlikely, that two valid users have the same IP address.
- Valid users are unlikely to make many mistakes entering their credentials.

The simulator makes the following assumptions about the hackers:

- The hackers try to avoid an account lockout by only testing a few username-password combinations rather than a full-blown dictionary attack (trying every password the hacker has in a dictionary of possible passwords that they maintain on every user). However, they don't add delays between their attempts.
- Since the hackers don't want to cause a denial of service, they limit the volume of their attacks and only make one attempt at a time.
- The hackers know the amount of accounts that exist in the system and have a good idea the format the usernames are in, but are guessing what they are exactly. They will choose to try to guess all 133 usernames or some subset of it.
- Each attack is standalone, meaning there is a single hacker acting for each attack.
- The hackers don't share information about which username-password combinations are correct.
- The attacks come randomly.
- Each hacker will use a single IP address, which is generated in the same way the valid user ones are. However, our simulator is capable of varying this IP address when `vary_ips=True` is passed to `simulate()`.
- Although highly unlikely, it is possible the hacker has the same IP address as a valid user. The hacker may even be a valid user.

## Performance

> There are some obvious performance improvements

> It takes **~900** ms to simulate 30 days data and the same in Python takes **15 seconds** on 11th Gen Intel(R) Core(TM) i7-1165G7 @ 2.80GHz - 16 GB RAM

> I know that there is room for lot of performance improvement on both sides but this comparison gives some idea of what is possible