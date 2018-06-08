import * as fs from "fs";
import {JsonConvert} from "json2typescript";

import {Idea, IdeaJson} from "../Idea";
import {BotProcess, BotStatus} from "../DaVinciBot";

export class LoadFileProcess extends BotProcess
{
    start(): void {
        this.status = BotStatus.NeedsInput;
    }

    handleInput(input: string): void {
        if (fs.existsSync(input)) {
            this.bot.startProcess(new LoadProcess(this.bot, this.rootIdea));
            this.bot.handleInput(fs.readFileSync(input, 'utf8'));
        }
        else {
            // TODO sometimes it might be error behavior if there's no file to
            // load
        }

        this.status = BotStatus.Idle;
    }
}

export class SaveFileProcess extends BotProcess
{
    start(): void {
        this.status = BotStatus.NeedsInput;
    }

    handleInput(input: string): void {
        this.bot.startProcess(new SaveProcess(this.bot, this.rootIdea));
        let output = this.bot.getOutput();
        fs.writeFileSync(input, output, 'utf8');
        this.status = BotStatus.Idle;
    }
}

export class LoadProcess extends BotProcess
{
    static converter = new JsonConvert();

    start() {
        this.status = BotStatus.NeedsInput;
    }

    handleInput(input: string): void {
        let jsonStart = input.indexOf('{');
        let countInput = input.substr(0, jsonStart);
        let jsonInput = input.substr(jsonStart);

        let jsonObject: IdeaJson = JSON.parse(jsonInput) as IdeaJson;
        console.log(jsonObject);
        let newRootIdea = LoadProcess.converter.deserializeObject(jsonObject, Idea);

        // Recursively load all the children as Idea objects
        for (let i = 0; i < jsonObject.children.length; ++i) {
            let childLoadProcess = new LoadProcess(this.bot, new Idea());
            this.bot.startProcess(childLoadProcess);
            console.log(JSON.stringify(jsonObject.children[i]));
            this.bot.handleInput(countInput + JSON.stringify(jsonObject.children[i]));
            newRootIdea.children[i] = childLoadProcess.rootIdea;
        }

        // FIXME if deserialization tests are failing it is probably because of
        // this line, which is necessary because ES5 doesn't define
        // Object.assign()
        this.rootIdea.become(newRootIdea);

        // Make sure TotalCount is properly set
        Idea.TotalCount = parseInt(countInput);

        this.status = BotStatus.Idle;
    }
}

export class SaveProcess extends BotProcess
{
    start() {
        this.status = BotStatus.HasOutput;
    }

    getOutput(): string {
        this.status = BotStatus.Idle;
        return Idea.TotalCount + JSON.stringify(this.rootIdea);
    }
}
