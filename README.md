# Treasury Program

The treasury programs serves as a variety of holding places for ZEE.

## Simple Treasury

There is exactly one possible treasury for every Solana address, with the respective solana address acting as authority. As the name implies, simple treasuries don't do anything fancy. There is only one type of simple treasury available currently, and that is `LOCKED`, a mode that only accepts funds but has no way of releasing it.

Simple treasuries will acquire additional functionality in the future.

## Vested Treasury

A vested treasury makes funds accessible over a period of time. The initialization parameters are:
* `amount`: The total amount of funds that are distributed
* `period`: The time (in seconds) of a single period
* `percentage`: The percentage of the total funds released every period

The treasury can be initialized without the funds being available up front. In that case, the beneficiary can claim everything in the account *up to* the maximum theoretical funds. This allows a vested treasury to be created and then have the funds minted directly into its fund address.

Multiple vested treasuries can be created for a single beneficiary.