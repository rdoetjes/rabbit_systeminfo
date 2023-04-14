# System Info

Is a Web Top, well only memory and CPU.

This was inspired by a video I came across looking into gathering system info for rust: https://www.youtube.com/watch?v=c_5Jy_AVDaM
So I figured let's implement it my way, using rocket on Rust.

I admit that I took his front-end stuff and tweaked it, because I hate front-end with a passion!
And I created my own version of the webservice. The idea is to have this running on my 4 PIs that have build agents running. And now I log in and 
run top/htop but I would like to just first see their business on a convenient web frontend.

I will add some process lists as well.

And maybe I will also do a WebSocket version like fasterthanlime did in his video. It's kind off a nicer way.
