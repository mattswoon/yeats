const { v4: uuidv4 } = require('uuid');
const Discord = require('discord.js');
client_id = process.env.CLIENT_ID;
token = process.env.TOKEN;
console.log(`Connecting with client_id=${client_id} and token=${token}`);
const client = new Discord.Client();

client.once('ready', () => {
  console.log('Connected and ready');
});

class Clue {
  constructor(user, text) {
    this.user = user;
    this.text = text;
  }

  toString() {
    return `"${this.text}" added by ${this.user.username}`;
  }

  showTo(user) {
    console.log(`Showing "${this.text}" to ${user.username}`);
    user.send(`Your clue is:\n\n\t\t${this.text}`);
  }
}

class Bowl {
  constructor() {
    this.unsolved = new Array;
    this.solved = new Array;
    this.per_person_limit = null;
  }

  summary() {
  }

  cluesFrom(user) {
    return this.unsolved
      .filter((c) => c.user === user)
      .concat(this.solved.filter((c) => c.user === user)); 
  }

  numClues() {
    return this.unsolved.length + this.solved.length;
  }

  addClue(clue) {
    this.unsolved.push(clue);
    console.log(`Added clue: ${clue}`);
  }

  toString() {
    var num_unsolved = this.unsolved.length;
    var num_solved = this.solved.length;
    var total_clues = num_unsolved + num_solved;
    return `${num_unsolved} out of ${total_clues} remain unsolved`;
  }
  
  shuffle() {
    shuffle(this.unsolved);
  }

  draw() {
    return this.unsolved.pop();
  }

  putBack(clue) {
    if (clue) {
      this.unsolved.push(clue);
      this.shuffle();
    };
  }

  makeSolved(clue) {
    this.solved.push(clue);
  }

  putAllBack() {
    while (this.solved.length) {
      clue = this.solved.pop();
      this.unsolved.push(clue);
    };
  }
}

class Players {
  constructor() {
    this.players = new Array;
    this.shows_for = new Map;
    this.awaiting_turn = new Array;
    this.had_turn = new Array;
  }

  addPlayer(user) {
    this.players.push(user);
    console.log(`Added ${user.username} to the game`);
  }

  resetQueue() {
    this.awaiting_turn = this.players.slice();
    this.makeTurnOrder();
    this.had_turn = new Array;
  }

  makeTurnOrder() {
    var shows_for = new Map;
    var queue = this.players.slice();
    shuffle(queue);
    var first_player = queue.pop();
    var performer = first_player;
    var guesser = null;
    while (queue.length) {
      guesser = queue.pop();
      shows_for.set(performer, guesser);
      performer = guesser;
    };
    shows_for.set(performer, first_player);
    this.shows_for = shows_for;
  }

  numWaiting() {
    return this.awaiting_turn.length;
  }
}

const rounds = {
  SENTENCE: 'sentence',
  WORD: 'word',
  CHARADES: 'charades'
}

const turnStatus = {
  READY: "ready",
  RUNNING: "running",
  FINISHED: "finished"
}

class Turn {
  constructor(performer, guesser, turn_status) {
    this.performer = performer;
    this.guesser = guesser;
    this.turn_status = turn_status;
    this.shown_clues = new Array;
    this.solved_clues = new Array;
    this.currently_solving = null;
    this.uuid = uuidv4();
  }

  isReady() {
    return this.turn_status === turnStatus.READY;
  }

  isFinished() {
    return this.turn_status === turnStatus.FINISHED;
  }

  isRunning() {
    return this.turn_status === turnStatus.RUNNING;
  }

  recap() {
    return 'Just to recap, these were the clues:\n\t\t' + this.solved_clues.map((c) => c.text).join('\n\t\t');
  }
}

class Game {
  constructor() {
    this.reset();
    this.turn_time = 90000;
  }

  reset() {
    this.round = rounds.SENTENCE;
    this.bowl = new Bowl();
    this.players = new Players();
    this.locked = false;
    this.turn = null;
    this.channel = null;
  }

  startGame(channel) {
    this.channel = channel;
    this.locked = true;
    this.bowl.shuffle();
    this.players.makeTurnOrder();
    this.players.resetQueue();
    channel.send(`Gather round folks, the game is about to begin!`);
  }

  restartGame() {
    this.round = rounds.SENTENCE;
    this.players.resetQueue();
    this.bowl.putAllBack();
  }

  nextPlayer() {
    return this.players.awaiting_turn[this.players.awaiting_turn.length - 1];
  }

  prepTurn() {
    if (this.players.numWaiting() == 0) {
      console.log('Queue empty, resetting it');
      this.players.resetQueue();
    };
    var performer = this.players.awaiting_turn.pop();
    var guesser = this.players.shows_for.get(performer);
    var turn = new Turn(performer, guesser, turnStatus.READY);
    this.turn = turn;
    this.channel.send(`Get ready ${performer}, you'll be performing to ${guesser} ` +
      `who will be guessing.\nRun \`!start-turn\` to begin.`);
    console.log(`Prepped turn\n\tPerformer: ${performer.username}\n` +
      `\tGuesser: ${guesser.username}\n` + 
      `\tTurn Status: ${turn.turn_status}`);
  }

  drawClue() {
    clue = this.bowl.draw();
    if (clue) {
      this.turn.currently_solving = clue;
      clue.showTo(this.turn.performer);
    } else {
      this.turn.currently_solving = null;
      this.endTurn(this.turn.uuid);
      this.nextRound();
    };
  }

  nextClue() {
    this.turn.solved_clues.push(this.turn.currently_solving);
    this.drawClue();
  }

  endTurn(uuid) {
    if ((uuid === this.turn.uuid) & this.turn.isRunning()) {
      console.log(`Ending turn with uuid=${uuid}`);
      this.turn.turn_status = turnStatus.FINISHED;
      var num_solved = this.turn.solved_clues.length;
      var recap = this.turn.recap();
      if (num_solved == 0) {
        this.channel.send(`Atrocious ${this.turn.performer}, you didn't get any!`);
      } else if (num_solved <=2 ) {
        this.channel.send(`Ooft ${this.turn.performer} that was rough. You solved ${num_solved}\n` + recap);
      } else if (num_solved <= 4) {
        this.channel.send(`Hey not bad ${this.turn.performer}, you solved ${num_solved}\n` + recap);
      } else {
        this.channel.send(`Nice work ${this.turn.performer}, you solved ${num_solved}\n` + recap);
      };
      this.bowl.putBack(this.turn.currently_solving);
      for (clue of this.turn.solved_clues) {
        this.bowl.makeSolved(clue);
      };
      this.players.had_turn.push(this.turn.performer);
      console.log(this.bowl.toString());
      var clues_left = this.bowl.unsolved.length;
      switch (clues_left) {
        case 0:
          this.channel.send('No clues left\nTo queue up the next turn run `!next-turn`');
          break;
        case 1:
          this.channel.send('Only one clue left!\nTo queue up the next turn run `!next-turn`');
          break;
        default:
          this.channel.send(`There are ${clues_left} clues left to solve\nTo queue up the next turn run \`!next-turn\``);
      }
    } else {
      console.log('Got an endTurn for an old turn');
    };
  }

  async runTurn(uuid) {
    console.log(`Starting turn with uuid=${uuid}`);
    this.turn.turn_status = turnStatus.RUNNING;
    this.drawClue()
    this.channel.send(`Ready ${this.turn.performer}? GO!`);
    await sleep(this.turn_time);
    this.endTurn(uuid);
  }

  nextRound() {
    switch(this.round) {
      case rounds.SENTENCE:
        this.round = rounds.WORD;
        this.bowl.putAllBack();
        this.channel.send('ROUND OVER! All the clues have gone back into the bowl ' +
          'and we start again.\nThis time the performer can only say *a single word*. ' +
          'They may say or do nothing else! No accents, no suggestive looks...\n' +
          'JUST :clap: ONE :clap: WORD :clap:');
        break;
      case rounds.WORD:
        this.round = rounds.CHARADES;
        this.bowl.putAllBack();
        this.channel.send('YOU\'VE JUST GONE AND BLOODY DONE IT! That\'s the end ' +
          'of the round folks. The clues are back in the bowl and we\'re ready to go again.\n' +
          'This time the performer has to do *charades*. They can say nothing, no words, ' +
          'no sound effects... nothing!');
        break;
      case rounds.CHARADES:
        this.channel.send('Well that\'s it comrades, game over. Game over man!');
        this.reset();
        break;
    }
  }

  enoughPlayers() {
    return this.players.players.length >= 2;
  }

  enoughClues() {
    return this.bowl.numClues() > 0;
  }

//    gameState() {
//    }
}

var game = new Game();

client.on('message', (message) => {
  if (message.channel.type === 'dm' & !message.author.bot & message.content.startsWith('!')) {
    command = message.content.split(' ')[0];
    switch (command) {
      case '!add-clue':
        if (!game.locked) {
          text = message.content.substring(command.length).trim();
          clue = new Clue(message.author, text);
          game.bowl.addClue(clue);
        } else {
          message.reply('Sorry the game has already started, you\'ll have to wait until the next one');
        };
        break;
      default:
        message.reply('Unknown command!');
    };
  };
  if (game.turn) {
    if ((message.author === game.turn.performer) & (game.turn.isRunning())) {
      // mark current clue as solved, and get the next one
      game.nextClue();
    };
  };
  if (message.channel.type === 'text' & !message.author.bot & message.content.startsWith('!')) {
    command = message.content.split(' ')[0];
    switch (command) {
      case '!add-players':
        // add players to the game
        if (!game.locked) {
          added = new Array;
          for (user of message.mentions.users.array()) {
            game.players.addPlayer(user);
            added.push(user.toString());
          };
          message.reply("Added " + added.join(', ') + " to the game");
        } else {
          message.reply("Sorry, can't add players to the game because it's already started");
        };
        break;
      case '!start-game':
        // starts the game
        if (game.locked) {
          message.reply('The game has already started');
        } else if (!game.enoughPlayers()) {
          message.reply('Not enough players yet');
        } else if (!game.enoughClues()) {
          message.reply('Still waiting on a few more clues');
        } else {
          game.startGame(message.channel);
          game.prepTurn();
        };
        break;
      case '!next-turn':
        if (game.locked) {
          if (game.turn.isFinished()) {
            game.prepTurn();
          } else {
            message.reply('Not ready to start a new turn yet');
          };
        } else {
          message.reply('The game hasn\'t started yet');
        };
        break;
      case '!start-turn':
        // starts a turn
        if (game.locked) {
          if (game.turn.isReady()) {
            game.runTurn(game.turn.uuid);
          } else {
            message.reply('Not ready to start a new turn yet');
          };
        } else {
          message.reply('Hold your horses, the game hasn\'t started yet');
        };
        break;
      case '!clue-summary':
        message.reply('This one\'s a work-in-progress');
        break;
      case '!list-players':
        message.reply('This one\'s a work-in-progress');
        break;
      case '!game-state':
        message.reply('This one\'s a work-in-progress');
        break;
      case '!restart-game':
        message.reply('This one\'s a work-in-progress');
        break;
      case '!reset-game':
        message.reply('This one\'s a work-in-progress');
        break;
      case '!help':
        message.reply('This one\'s a work-in-progress');
        break;
      default:
        author = message.author;
        message.channel.send(`Sorry ${author}, I didn't understand that`);
    };
  };
});

/**
 * Shuffles array in place.
 * @param {Array} a items An array containing the items.
 */
function shuffle(a) {
    var j, x, i;
    for (i = a.length - 1; i > 0; i--) {
        j = Math.floor(Math.random() * (i + 1));
        x = a[i];
        a[i] = a[j];
        a[j] = x;
    }
    return a;
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

client.login(token);
