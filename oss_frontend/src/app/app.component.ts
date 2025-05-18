import {Component, OnDestroy, OnInit} from '@angular/core';
import {TranslateService} from '@ngx-translate/core';
import {NavigationEnd, Router, RouterOutlet} from '@angular/router';
import {Subscription} from 'rxjs';

@Component({
    selector: 'app-root',
    template: '<router-outlet></router-outlet>',
    imports: [RouterOutlet],
})
export class AppComponent implements OnInit, OnDestroy {
    /** Subscriptions of the component. */
    private subscriptions = new Subscription();

    constructor(private router: Router, private translate: TranslateService) {
    }

    ngOnInit(): void {
        this.initializeLanguage();
        this.scrollTopWhenNavigationChanged();
    }

    ngOnDestroy(): void {
        this.subscriptions.unsubscribe();
    }

    private initializeLanguage(): void {
        const savedLang = localStorage.getItem('preferred_language');
        if (savedLang) {
            this.translate.use(savedLang);
        } else {
            const browserLang = navigator.language.split('-')[0];
            if (['en', 'es'].includes(browserLang)) {
                this.translate.use(browserLang);
                localStorage.setItem('preferred_language', browserLang);
            } else {
                this.translate.use('en');
                localStorage.setItem('preferred_language', 'en');
            }
        }
    }

    private scrollTopWhenNavigationChanged(): void {
        const subscription = this.router.events.subscribe(evt => {
            if (!(evt instanceof NavigationEnd)) {
                return;
            }
            window.scrollTo(0, 0);
        });
        this.subscriptions.add(subscription);
    }
}