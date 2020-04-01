# epidemic
rust simulation of an epidemic

There's a compiled windows 64bit version and Linux (will build a Raspberry Pi version
later)

The idea of this is to play around with profiles of ages, susceptibilty to disease
(by age), amount of moving about (by age), infectiousness, time of increasing severity,
number of people, size of grid, number and sizes of cities/towns. (and other things).

The simulation works by moving the people around in a random walk with a bias towards
heading home. If the person mobility is above 2 then there is a chance that the person
will jump to a city. There is also a (higher) probability they will jump home.

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

    # lockdown starts and ends..
    "lockdown_start": 70,
    "lockdown_end": 100,

    # number of periods after first infection that the disease worsens
    "to_peak": 6,

    # amount of infection from visiting people before grid cell becomes contaminated
    "cell_threshold": 7,

    # number of periods before infected person can pass on infection to grid cell
    "noninfective": 2,

    # save an image in the folder ./frames/
    "save_images": true,

    # size of images in pixels
    "image_size": 1000
}
