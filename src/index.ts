import {Idea} from "./Idea";
import {DaVinciBot} from "./DaVinciBot";
import {BotStatus} from "./BotProcess";
import {AddIdeaProcess} from "./processes/AddIdeaProcess";
import {PlanIdeaProcess} from "./processes/PlanIdeaProcess";
import {WorkOnIdeaProcess} from "./processes/WorkOnIdeaProcess";
import {LoadProcess, SaveProcess, LoadFileProcess, SaveFileProcess} from "./processes/Serialization";
import * as readlineSync from "readline-sync";
import * as os from "os";
import * as path from "path";

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

let rootIdea = new Idea();
let bot: DaVinciBot = new DaVinciBot();

// Automatically load state
bot.startProcess(new LoadFileProcess(bot, rootIdea));
bot.handleInput(path.join(os.homedir(), '.davinci.json'));

bot.startProcess(new WorkOnIdeaProcess(bot, rootIdea));

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

    console.log(bot.currentProcess);
    console.log(bot.status);
}

bot.startProcess(new SaveFileProcess(bot, rootIdea));
bot.handleInput(path.join(os.homedir(), '.davinci.json'));
