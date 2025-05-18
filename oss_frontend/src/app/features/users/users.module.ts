import {NgModule} from '@angular/core';
import {DatePipe} from '@angular/common';
import {provideRouter} from '@angular/router';
import {routes} from './users.routes';

@NgModule({
  providers: [
    DatePipe, provideRouter(routes),
  ],
})
export class UsersModule {
}
