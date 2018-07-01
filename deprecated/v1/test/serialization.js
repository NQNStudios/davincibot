'use strict';

var expect = require('chai').expect;
var idea = require('../dist/Idea.js');
var davincibot = require('../dist/DaVinciBot.js');
var addIdeaProcess = require('../dist/processes/AddIdeaProcess.js');
var serialization = require('../dist/processes/Serialization.js');

function createTestIdea() {
  idea.Idea.TotalCount = 0;
  var rootIdea = new idea.Idea('root has a name!');
  rootIdea.addChild('test1');
  rootIdea.addChild('test2');
  rootIdea.addChild('test3');
  rootIdea.children[2].description = 'test3 has a description';
  return rootIdea;
}

describe('JSON serialization test', () => {
  var bot = new davincibot.DaVinciBot();
  var rootIdea = createTestIdea();

  bot.startProcess(new serialization.SaveProcess(bot, rootIdea));
  var json = bot.getOutput();


  it('Should serialize the idea count', () => {
    // TODO parse and assert totalCount is 3
    // var totalCount = 
  });

  var jsonObject = JSON.parse(json.substring(1));

  it('should serialize the root name', () => {
    expect(jsonObject.name).to.equal('root has a name!');
  });

  it('Should serialize all children', () => {
    expect(jsonObject.children.length).to.equal(3);
  });
  it("Should serialize all children's names", () => {
    expect(jsonObject.children[0].name).to.equal("test1");
    expect(jsonObject.children[1].name).to.equal("test2");
    expect(jsonObject.children[2].name).to.equal("test3");
  });
  // TODO should serialize descriptions
  // TODO should serialize all tags
});

describe('File serialization test', () => {
  var bot = new davincibot.DaVinciBot();
  var rootIdea = createTestIdea();

  bot.startProcess(new serialization.SaveFileProcess(bot, rootIdea));
  bot.handleInput('.serial-test-1.json');

  var newRootIdea = new idea.Idea();
  bot.startProcess(new serialization.LoadFileProcess(bot, newRootIdea));
  bot.handleInput('.serial-test-1.json');

  it('Should deserialize the global idea count', () => {
    expect(idea.Idea.TotalCount).to.equal(4);
  });
  it('Should deserialize all children', () => {
    expect(newRootIdea.children.length).to.equal(3);
  });
  it('Should preserve the Idea names after deserialization', () => {
    expect(newRootIdea.children[0].name).to.equal('test1');
    expect(newRootIdea.children[1].name).to.equal('test2');
    expect(newRootIdea.children[2].name).to.equal('test3');
  });
  it('Should deserialize children as fully-featured Idea objects', () => {
    expect(() => {newRootIdea.children[0].addChild('Test4');}).to.not.throw();
  });

  it('Should deserialize children with descriptions intact', () => {
    expect(newRootIdea.children[2].description).to.equal("test3 has a description");
  });
  // TODO should preserve progress values
  // TODO should preserve all tags
  // TODO should preserve all moment.Duration objects -- this definitely
  // shouldn't work yet
});
