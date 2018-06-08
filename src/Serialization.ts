import * as fs from "fs";
import {JsonConvert} from "json2typescript";

import {Idea} from "./Idea";
import {BotProcess, BotStatus} from "./DaVinciBot";

export class LoadFileProcess extends BotProcess
{
    description(): string { return  'Load ideas from a file.'; }

    start(): BotStatus {
        return BotStatus.NeedsInput;
    }

    handleInput(input: string): BotStatus {
        if (fs.existsSync(input)) {
            return new LoadProcess().init(this.rootIdea).handleInput(fs.readFileSync(input, 'utf8'));
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

    start(): BotStatus {
        return BotStatus.NeedsInput;
    }

    handleInput(input: string) {
        let output = new SaveProcess().init(this.rootIdea).getOutput()[0];
        fs.writeFileSync(input, output);
        return BotStatus.Idle;
    }
}

export class LoadProcess extends BotProcess
{
    static converter = new JsonConvert();

    description(): string { return 'Load ideas from a JSON string.'; }

    start(): BotStatus {
        return BotStatus.NeedsInput;
    }

    handleInput(input: string) {
        let jsonStart = input.indexOf('{');
        let countInput = input.substr(0, jsonStart);
        let jsonInput = input.substr(jsonStart);

        let jsonObject: object = JSON.parse(jsonInput);
        let newRootIdea = LoadProcess.converter.deserializeObject(jsonObject, Idea);
        this.rootIdea.become(newRootIdea);

        // Make sure TotalCount is properly set
        Idea.TotalCount = parseInt(countInput);

        return BotStatus.Idle;
    }

    finish(): void {
        // TODO: Don't always do this
        console.log(this.rootIdea.children);
    }
}

export class SaveProcess extends BotProcess
{
    description(): string { return 'Save ideas to a JSON string.'; }

    start(): BotStatus {
        return BotStatus.HasOutput;
    }

    getOutput(): [string, BotStatus] {
        return [Idea.TotalCount + JSON.stringify(this.rootIdea), BotStatus.Idle];
    }
}
