import {NgModule} from '@angular/core';
import {provideRouter} from '@angular/router';
import {routes} from './users.routes';

@NgModule({
  providers: [provideRouter(routes)],
})
export class UsersModule {
}
