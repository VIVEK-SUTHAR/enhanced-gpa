# Enhanced GPA

**Enhanced GPA** is a Rust program that processes Solana accounts by adding detailed token information, including token prices, to the account data. It enhances Get Program Accounts (GPA) results by leveraging BirdEye's APIs to fetch token prices and additional metadata.

## Features

- **Account Processing**: Returns all accounts owned by the provided program `Pubkey`.
- **Token Information**: Extracts and adds token details such as the token name, symbol, and balance.
- **Token Prices**: Fetches and adds real-time token prices.
- **Currency Conversion**: Supports converting token prices to multiple currencies, including:
  - USD (United States Dollar)
  - EUR (Euro)
  - GBP (British Pound Sterling)
  - JPY (Japanese Yen)
  - INR (Indian Rupee)
  - RUB (Russian Ruble)
- **Enhanced GPA Results**: Outputs enriched account data with token information and prices in the JSON format;

## Endpoint:

### `GET /getTokens/{address}`

**Description:**
Fetches all token holdings  for a given Solana account address. Returns detailed token information including token name, balance, and price converted to the specified currency.

**Path Parameters:**

- **`address`** (required): The Solana account address in base58 format for which token information is to be retrieved.

**Query Parameters:**

- **`currency`** (optional): The target currency for price conversion. Defaults to USD if not specified.
- **`sortbyvalue`** (optional): sorts the token list by the value

## TODO

- **Sync Token List**: Implement a feature to synchronize the token list periodically to ensure up-to-date token metadata.
- **Batch Price Fetching**: Optimize price fetching by batching requests to the BirdEye API.
- **Currency Price Sync**: Implement a feature to synchronize the currency prices periodicall.
