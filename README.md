# README

## Assumptions and comments:
1. Once an account is frozen, no further transactions should be processed
2. Since this is just a cli, which in nature will only run and terminate, there are limitations of the code to match
3. Assuming you can't re-dispute a chargebacked or resolved transaction, once we have a resolve or chargeback we could remove those transactions, but I figured we want to keep them for historical reasons

## Implementation
All the sauce is in `Account`, it keeps track of a client's account (money, frozen).
It has a function called `process` which processed a transaction and updates the account.
Account also keeps a copy of all past transactions in order to support dispute, resolve and rollback

## Testing strategy
No unit tests, I instead opted for testing TransactionEngine as an isolated unit since it has all the domain logic  
I'm not die-hard opposed to them, I just like having the bulk of my tests in a way that also the domain experts can easier understand them.  
For testing main I would probably write a dummy transaction engine that just expected a certain input and always gave the same output  
and write a test that actually compiled the binary and ran it with a known input to separate testing main from TransactionEngine  
That way we can write all of the edge cases using a nice code-api and have fever tests for main (which is not very interesting)

## Changes if this had been a real system
I'd assume that the transactions is some type of data stream, like a topic in a message queue.  
Then I'd probably would've written a data pipeline (batch or streaming depending on requirements)  
that ingests transactions, stores them in some long term file store(S3/GCS) and updates a view/projection stored in a database,  
I would default to postgres unless it didn't serve our needs (load/size etc)  
For scalability, the data can be sharded on client id.

It would ofc be possible to do this as a traditional backend service, but the same ideas kinda apply  
store all events to some long term storage for historical reasons  
read each transaction and update the account in some DB
