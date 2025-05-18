import {NgModule} from '@angular/core';
import {provideRouter} from '@angular/router';
import {routes} from './devices.routes';

@NgModule({
  providers: [provideRouter(routes)],
})
export class DevicesModule {
}
