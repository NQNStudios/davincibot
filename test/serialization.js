'use strict';

var expect = require('chai').expect;
var idea = require('../dist/Idea.js');
var davincibot = require('../dist/DaVinciBot.js');
var addIdeaProcess = require('../dist/processes/AddIdeaProcess.js');
var serialization = require('../dist/processes/Serialization.js');

describe('Serialization test', () => {
  var bot = new davincibot.DaVinciBot();
  var rootIdea = new idea.Idea();
  bot.startProcess(new addIdeaProcess.AddIdeaProcess(bot, rootIdea));
  bot.getOutput();
  bot.handleInput('test1');
  bot.handleInput('test2');
  bot.handleInput('test3');
  bot.handleInput('quit');


  bot.startProcess(new serialization.SaveFileProcess(bot, rootIdea));
  bot.handleInput('.serial-test-1.json');

  var newRootIdea = new idea.Idea();
  bot.startProcess(new serialization.LoadFileProcess(bot, newRootIdea));
  bot.handleInput('.serial-test-1.json');

  it('Should preserve the Idea names after deserialization', () => {
    expect(newRootIdea.children[0].name).to.equal('test1');
    expect(newRootIdea.children[1].name).to.equal('test2');
    expect(newRootIdea.children[2].name).to.equal('test3');
  });

});
