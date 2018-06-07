import {DaVinciBot, BotStatus, BotState} from "./DaVinciBot";
import {AddIdeaState} from "./AddIdeaState";
import * as readlineSync from "readline-sync";

// TODO turn this into a test script
/*import { JsonConvert } from "json2typescript";*/
/*let recursiveIdea = new Idea("Top-level idea");*/

/*let middleIdea = recursiveIdea.addChild("middle-level child");*/

/*let finalIdea = middleIdea.addChild("bottom-level child");*/

/*let converter = new JsonConvert();*/

/*let jsonString = JSON.stringify(recursiveIdea);*/

/*let deserializedIdea = Idea.ParseString(jsonString);*/

/*console.log(jsonString);*/
/*console.log(deserializedIdea);*/

let bot: DaVinciBot = new DaVinciBot();

bot.addState('AddIdeaState', new AddIdeaState());

bot.startState('AddIdeaState');

while (bot.status !== BotStatus.Idle) {
    switch (bot.status) {
        case BotStatus.HasOutput:
            console.log(bot.getOutput());
            break;
        case BotStatus.NeedsInput:
            let input = readlineSync.prompt();
            bot.handleInput(input);
            break;
    }
}

bot.finishCurrentState();
