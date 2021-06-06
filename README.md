# Ethereum gateway

This contracts allow users with Ethereum wallets to proxy their requests into NEAR using EIP-712.

Basic design:
 - gateway contract faciliates the account creation, validation of EIP-712 messages.
 - proxy contract is minimal code deployed on the users account that proxies requests from gateway.

