# yeats
I call it "irish" but I don't know what you call it - it's a game, now you can play it on discord.

Everyone puts clues into a bowl, then take turns performing the clues to someone else. There's 3 rounds
 1 - Use any words except what's written on the clue
 2 - Use only one word that's not written on the clue
 3 - Charades
 
All rounds use the same set of clues and they go back into the bowl at the end of each round.

This is my discord bot to run the game so you can play it over the internet during a global pandemic when you can't go round to your friends' houses.

# How it works?
Do the usual things to add a bot to your discord channel. This guy is run off a few commands

## Submitting clues
*Direct message* the bot with
`!add-clue <TEXT>` 
to add `<TEXT>` as a clue

## Adding players
In a text channel type
`!add-player @playername1 @playername2 ...`
To add players to the game

## Starting the game
When there's enough players and clues, type into a text channel
`!start-game`
and follow instructions from there.

## Turns
Everyone is assigned a single person to perform to, when it's your turn you'll be performing your clue to that other person and them alone. Only when they guess correctly can you move on to the next clue. This one-at-a-time rule is to deal with the problems with having many people yelling over voice/video chat at the same time.

Every time the turn queue resets (everyone has a turn) the performer-performee assignment is randomly regenerated.

When it's your turn to perform, the bot with direct message you a clue. Reply to the bot or a text channel with *anything* to get the next clue - turns are too exciting to have to bother with typing a specific command!

At the end of your turn the bot will recap which clues you solved - the last one shown to you is put back into the bowl. The recap message will be **REDACTED** after a certain delay, so you can't just scroll up the channel to remind yourself what clues there are.

# Disclaimer
I've never written javascript before this, and I don't care if you think it's bad.
