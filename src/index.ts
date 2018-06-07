import {DaVinciBot, BotStatus, BotProcess} from "./DaVinciBot";
import {AddIdeaProcess} from "./AddIdeaProcess";
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
bot.addProcess('AddIdeaProcess', new AddIdeaProcess());

while (true) {
    // TODO print all available processes
    let availableProcesses = bot.processes;
    for (let name in availableProcesses) {
        let process = availableProcesses[name];
        console.log(`${name} - ${process.description}`);
    }

    console.log(`Choose which process to run, or type 'quit'`);
    let process = readlineSync.prompt();

    if (process === 'quit') {
        break;
    }

    bot.startProcess(process);

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

    bot.finishCurrentProcess();
}
