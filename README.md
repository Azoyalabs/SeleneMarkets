# Contract Template CosmWasm  

## Overview  

Template for CosmWasm 1.0 contract development. 

This addresses a common pattern in contracts: there is usually a permissioned side to the contract, that cannot be accessed by the user.  
To reflect this, we seperate permissioned messages into a AdminExecuteMsg and route between AdminExecuteMsg and non-permissioned variant in the contract.rs file.  

We also route all query messages to a router with dedicated functions in contract_query.rs. We also box reponses and use the erased-serde Serialize trait to avoid the usual repeated Ok(to_binary(response_struct)) in query functions.  

Finally, we include a basic setup for cw-multi-test to enable multi-contract testing from the get go, and provide a sample contract setup using it. 
Basic testing does NOT support multi contract testing (and neither execution of sub messages), while this allows to handle replies, instantiation and all other contract calls emitted by the tested contract.  


To note: this is not an OOP approach (like what can be found in cw721) so testing does get more bulky but it will allow simulating an actual blockchain environment.  

