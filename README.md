# Provisional Treasury Program

This program serves as the provisional treasury while the spec for the treasury program is being written. It serves as a holding for the user- and zoints-communities treasury funds. It is currently deposit only with no way to transfer the tokens out again. All non-system transactions are performed in "ZEE" which is an SPL token that is defined upon program initialization.

## Program Parameters

### Settings
* `token`: The SPL token that fees are paid in (This should be the ZEE token)
* `fee_recipient`: The associated account (of the `token` mint) that will receive fees
* `price_authority`: The pubkey of the account that can set the fees and fee recipient
* `launch_fee_user`: Cost of launching a user treasury (in ZEE)
* `launch_fee_zoints`: Cost of launching a zoints treasury (in ZEE)

## Instructions

### Initialize

Initializes the `Settings` account with the specified parameters

### Create User Treasury

Creates a user treasury account at program address `[b"user", <address of creator>]`. The treasury's associated ZEE address needs to be generated outside of this program.

### Create Zoints Treasury

Creates a zoints treasury account at program address `[b"zoints", <bytes of the given name>]`. The treasury's associated ZEE address needs to be generated outside of this program.

### Update Fees

Allows the `price_authority` to update `fee_recipient`, `launch_fee_user`, `launch_fee_zoints`.
