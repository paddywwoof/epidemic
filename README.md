# epidemic
rust simulation of an epidemic

There's a compiled windows 64bit version and Linux (will build a Raspberry Pi version
later)

To run it you need to change the current directory to the one with the executable
file and the json file.

The idea of this is to play around with profiles of ages, susceptibilty to disease
(by age), amount of moving about (by age), infectiousness, time of increasing severity,
number of people, size of grid, number and sizes of cities/towns. (and other things).

The simulation works by moving the people around in a random walk with a bias towards
heading home. If the person mobility is above 2 then there is a chance that the person
will jump to a city. There is also a (higher) probability they will subsequently jump home.

You can change the behaviour of the simulation by editing the constants.json file
which is loaded each time the program runs:
(can't have comments in json files so use this as reference)

    {
        # the size of the population
        "n": 200000,

        # the size of the grid
        "sz": 5000,

        # age distribution 0=>first decade 0-9, 1=>10-19, 2=>20-29 etc
        # so there are likely to be three times as many 30-39 as 70-79
        "age_dist": [0, 0, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 5, 5, 6, 7],

        # there is one line for each decade in the age distribution
        # the first column is mobility to start with
        # the second col is mobility during lockdown
        # the third col is resistance to the virus
        "age_vals": [[2, 0, 14],
                    [4, 0, 13],
                    [6, 0, 12],
                    [3, 1, 12],
                    [3, 1, 11],
                    [4, 1, 11],
                    [5, 0, 11],
                    [2, 0, 10]],

        # city w,h in grid units. Each entry is a city/town/village
        "city_sizes": [40, 20, 20, 20,
                    10, 10, 10, 10, 10, 5, 5, 5, 5, 5,
                    5,  5,  5,  1,  1, 1, 1, 1, 1, 1, 1, 1, 1],

        # number of infected people day 0
        "start_seed": 10,

        # number of simulation periods
        "n_steps": 200,

        # list of lockdown starts and ends.
        # must match i.e same number and end after or equal to start and before
        # next start
        "lockdown_start": [40, 70, 100, 130, 160],
        "lockdown_end": [60, 90, 120, 150, 180],

        # number of periods after first infection that the disease worsens
        "to_peak": 6,

        # amount of infection from visiting people before grid cell becomes contaminated
        "cell_threshold": 7,

        # what level of mobility is required before this age band might travel to
        # another city
        "jump_threshold": 2,

        # number of periods before infected person can pass on infection to grid cell
        "noninfective": 2,

        # save a sequence of images in the folder ./frames/
        "save_images": true,

        # size of images in pixels
        "image_size": 1000
    }

There are probably lots of non-realistic features to this simulation, however it
does give a feeling for the impact of travel restrictions on the spread of the
disease and the final death toll. (Spoiler: the final figures generally remain
the same - just takes longer to get there)