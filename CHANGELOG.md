# Changelog

What changed in each release, written for the people who use easy-ocpp.

## 0.4.0 (1 September 2026)

**Your charging sessions can now stop on their own.** Set a target amount of
energy, a running time, or both, and the wallbox is told to stop as soon as the
first of them is reached. You no longer have to come back and unplug at the
right moment. Every person can store their own defaults, so the limit applies
to each new session without being set again; an administrator can store them on
someone's behalf. While a session is running the values can still be changed,
and the timer then simply means "stop in this many minutes from now".

**Employees have their own page.** Someone who charges at your wallboxes signs
in and lands on a page showing what is running on their own chips right now:
kilowatt-hours so far, current power, state of charge if the car reports it.
From there they can change the limits or end the session themselves. They also
see their own charging history, can export it, and can pull their own monthly
report. What they cannot see is anyone else's charging, the wallboxes, the
chips, or the user administration. Those stay with the administrator.

**Employees and logins are one thing again.** Until now a person existed twice:
once as an employee record and once as a login account, and the two were only
loosely connected. That is over. A person is a user, and their chips, sessions
and limits belong to that one account. Existing employees are carried over
automatically. Anyone who had no login gets an account without a password,
which means their charging keeps being recorded but they cannot sign in until
an administrator gives them a password under "Users".

**Chips are no longer ambiguous.** A chip either belongs to a person or it is a
guest chip. Previously you had to set the owner and the category separately,
which allowed nonsense such as a guest chip assigned to an employee. The
category now follows from the assignment. If you built a stand-in person called
"Gast" to work around this, their chips become real guest chips during the
update and the stand-in is switched off.

**The program is now called easy-ocpp.** The old name had the letters of the
protocol swapped; it is spelled OCPP.

### Updating

Copy the new program over the old one and restart. Then delete the leftover
`easy-occp.exe`, otherwise two programs sit in the folder. If you run it as a
service, point the service at the new name.

Your database is kept. If only the old `data\easy-occp.db` is there, the
program keeps using it and says so in the log at startup. To switch to the new
name, stop the program, rename the file to `easy-ocpp.db` and start it again.

Everyone has to sign in once more, because the session cookie was renamed too.

## 0.3.1 (19 August 2026)

**Fixed: some installations refused to start after updating.** The program
stopped with a message about a modified migration, even though nothing about
the database had changed. This affected databases that had been created with a
different build than the one being installed. Those installations start
normally again.

## 0.3.0 (18 August 2026)

**The interface speaks four languages.** German, English, French and Spanish.
The language is picked from your browser settings and can be changed at any
time in the header.

**Releases are published automatically.** Ready-to-run packages for Windows and
Linux are built for every version and can be downloaded from the releases page.

## 0.2.0 (18 August 2026)

**You can watch a charging session as it happens.** The cockpit and the wallbox
page show how many kilowatt-hours have gone into the car, how much power is
flowing right now and, if the wallbox reports it, the state of charge. The
display refreshes every ten seconds by itself.

**Wallboxes are set up for you.** When a wallbox connects, the server tells it
to report meter readings every 30 seconds. If your wallbox does not report
power at all, it is worked out from the meter readings instead. The interval
can be changed in `config.toml`.
