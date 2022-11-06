# Cache ⚡
Cache is a software component that stores data in the memory so that future requests for that data can be served faster. The data stored in the cache
is called cache entry. The cache entry is stored in the cache memory and is associated with a key. The key is used to retrieve the cache entry from
the cache. The cache entry can be invalidated by removing it from the cache or by updating it. The cache entry can also be evicted from the cache if
the cache is full and a new entry needs to be added.

## Caching in Rustbase
Rustbase uses a cache to store the data that is frequently accessed. The cache is implemented using the LRU (Least Recently Used) algorithm. The LRU
algorithm is a cache eviction policy that removes the least recently used cache entry when the cache is full and a new entry needs to be added.
The LRU algorithm is implemented using a doubly linked list and a hash map. The doubly linked list is used to store the cache entries in the order of
their last access time. The hash map is used to store the cache entries in the order of their keys. The hash map is used to retrieve the cache entry
from the cache using the key. The doubly linked list is used to update the last access time of the cache entry when it is accessed. The doubly linked
list is also used to remove the least recently used cache entry when the cache is full and a new entry needs to be added.

### Reference
1. [Wikipedia](<https://en.wikipedia.org/wiki/Cache_(computing)>)