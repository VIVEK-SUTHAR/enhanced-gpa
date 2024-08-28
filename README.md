# Enhanced GPA

**Enhanced GPA** is a Rust program that processes Solana accounts by adding detailed token information, including token prices, to the account data. It enhances Get Program Accounts (GPA) results by leveraging BirdEye's APIs to fetch token prices and additional metadata.

## Features

- **Account Processing**: Returns all accounts owned by the provided program `Pubkey`.
- **Token Information**: Extracts and adds token details such as the token name, symbol, and balance.
- **Token Prices**: Fetches and adds real-time token prices using BirdEye's APIs.
- **Enhanced GPA Results**: Outputs enriched account data with token information and prices in the JSON format;

## TODO

- **Sync Token List**: Implement a feature to synchronize the token list periodically to ensure up-to-date token metadata.
- **Batch Price Fetching**: Optimize price fetching by batching requests to the BirdEye API.
