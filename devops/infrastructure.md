# Infrastructure

# DevOps Server
## System wide stuff
>Minio Server - file system

>Redis Server

>MQTT Server

>PostgreSQL 

>Nginx

>Internal docs

## Build server specific

> Git server - gogs  

>> Post push build compile and run verification _cargo test_ or otherwise

>>>Notification of build verification status

>>>>Success build

>>>>>Deploy updates to the application server depending on the branch (dev -> dev.api.application.com, prod -> api.application.com)

>>>>>Update production enclosure and schematic files to relevant file servers 

>>>>>Run post processing for marketing material and store to relevant file system

>>>>>Deploy updates to docs

>>>>>Build and verify kicad schematics and treat them like a code break alert

>>>Nightly job folder
>>>>cron tab file to keep track of scheduled jobs to run
>>>>>nightly end to end testing
>>>>>nightly rebuild of all projects and status alering

> Rust compiler

> Docker
    

# Application Server
>Nginx

>Docker

>>Marketing service

>>Hub as a SaaS web service
