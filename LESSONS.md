Some helpful lessons learned about working with PDFs:

- Favor `Result<()>` + `?` over `assert!` where possible. 
  - **WHY:** Acrobat does a lot of graceful error handling. Just because one object is unrenderable doesn't mean the whole program should crash! And if we do need it to crash, it's easier to come back later and convert to an assertion.