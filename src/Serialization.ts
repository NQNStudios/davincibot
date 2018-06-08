import * as fs from "fs";
import {JsonConvert} from "json2typescript";

import {Idea} from "./Idea";
import {BotProcess, BotStatus} from "./DaVinciBot";

export class LoadFileProcess extends BotProcess
{
    description(): string { return  'Load ideas from a file.'; }

    start(rootIdea: Idea): BotStatus {
        return BotStatus.NeedsInput;
    }

    handleInput(input: string, rootIdea: Idea): BotStatus {
        if (fs.existsSync(input)) {
            return new LoadProcess().handleInput(fs.readFileSync(input, 'utf8'), rootIdea);
        }
        else {
            // TODO sometimes it might be error behavior if there's no file to
            // load
            return BotStatus.Idle;
        }
    }
}

export class SaveFileProcess extends BotProcess
{
    description(): string { return  'Save ideas to a file.'; }

    start(rootIdea: Idea): BotStatus {
        return BotStatus.NeedsInput;
    }

    handleInput(input: string, rootIdea: Idea) {
        let output = new SaveProcess().getOutput(rootIdea)[0];
        fs.writeFileSync(input, output);
        return BotStatus.Idle;
    }
}

export class LoadProcess extends BotProcess
{
    static converter = new JsonConvert();

    description(): string { return 'Load ideas from a JSON string.'; }

    start(rootIdea: Idea): BotStatus {
        return BotStatus.NeedsInput;
    }

    handleInput(input: string, rootIdea: Idea) {
        let jsonStart = input.indexOf('{');
        let countInput = input.substr(0, jsonStart);
        let jsonInput = input.substr(jsonStart);

        let jsonObject: object = JSON.parse(jsonInput);
        let newRootIdea = LoadProcess.converter.deserializeObject(jsonObject, Idea);
        rootIdea.become(newRootIdea);

        // Make sure TotalCount is properly set
        Idea.TotalCount = parseInt(countInput);

        return BotStatus.Idle;
    }

    finish(rootIdea: Idea): void {
        // TODO: Don't always do this
        console.log(rootIdea.children);
    }
}

export class SaveProcess extends BotProcess
{
    description(): string { return 'Save ideas to a JSON string.'; }

    start(rootIdea: Idea): BotStatus {
        return BotStatus.HasOutput;
    }

    getOutput(rootIdea: Idea): [string, BotStatus] {
        return [Idea.TotalCount + JSON.stringify(rootIdea), BotStatus.Idle];
    }
}
